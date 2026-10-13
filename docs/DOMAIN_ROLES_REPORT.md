# Domain Roles Report — pkcore

| | |
|---|---|
| Subject | `pkcore` 0.15.4 — every top-level `pub` type under `src/` |
| Commit | `53345346`, branch `main` |
| Date | 2026-10-11 |
| Method | `/dddocs list`: `python3 ~/.claude/skills/dddocs/scripts/check_dddocs.py --list .` |
| Scope | structs, enums, unions, traits and type aliases. Free functions and methods are not listed. `#[cfg(test)]` items are skipped |
| Kind | `struct(T)` / `type(T)` names the inner type `T` of a struct with exactly one field (tuple or named), or the target of a type alias. Every other shape shows the bare kind |
| Note | The role is shown as written in the `# Domain` doc block. This report does not check that the role is valid. Use `/dddocs check` for that |

## Summary

- **254** `pub` types
- **195** have a `# Domain` role (76%)
- **59** have no role
- **59** wrap one inner type: a struct with one field, or a type alias

### By role

| Role | Types |
|------|------:|
| value object | 160 |
| cohesive mechanism | 17 |
| read model | 6 |
| action (DDD: command) | 3 |
| consistency boundary (DDD: aggregate) | 3 |
| specification | 2 |
| translator (DDD: anticorruption layer) | 2 |
| domain event | 1 |
| domain error | 1 |
| *(none)* | 59 |

### By kind

| Kind | Types |
|------|------:|
| `struct` | 175 |
| `enum` | 57 |
| `trait` | 17 |
| `type` | 5 |

## Types with no role

| Type | Kind | Location |
|------|------|----------|
| `SolverError` | `enum` | `src/analysis/gto/solver.rs:117` |
| `SolverCache` | `struct(PathBuf)` | `src/analysis/gto/solver_cache.rs:190` |
| `Versus` | `struct` | `src/analysis/gto/vs.rs:22` |
| `PlayOut` | `trait` | `src/analysis/mod.rs:32` |
| `Fg` | `struct(T)` | `src/analysis/nubibus.rs:26` |
| `Cyan` | `struct` | `src/analysis/nubibus.rs:32` |
| `LightBlack` | `struct` | `src/analysis/nubibus.rs:33` |
| `Yellow` | `struct` | `src/analysis/nubibus.rs:34` |
| `LightBlue` | `struct` | `src/analysis/nubibus.rs:35` |
| `LightRed` | `struct` | `src/analysis/nubibus.rs:36` |
| `Green` | `struct` | `src/analysis/nubibus.rs:37` |
| `LightGreen` | `struct` | `src/analysis/nubibus.rs:38` |
| `Magenta` | `struct` | `src/analysis/nubibus.rs:39` |
| `Reset` | `struct` | `src/analysis/nubibus.rs:40` |
| `Nubificus` | `struct` | `src/analysis/nubibus.rs:48` |
| `Pluribus` | `struct` | `src/analysis/nubibus.rs:763` |
| `PluribusEvent` | `enum` | `src/analysis/nubibus.rs:1460` |
| `StatsRegistry` | `struct` | `src/analysis/player_stats.rs:277` |
| `PlayerStatsStore` | `trait` | `src/analysis/player_stats_store.rs:63` |
| `YamlPlayerStatsStore` | `struct(PathBuf)` | `src/analysis/player_stats_store.rs:117` |
| `PlayerWins` | `struct(Wins)` | `src/analysis/player_wins.rs:14` |
| `IndexCardMap` | `struct` | `src/analysis/store/bcm/index_card_map.rs:43` |
| `Connect` | `struct(Connection)` | `src/analysis/store/db/sqlite.rs:3` |
| `Sqlable` | `trait` | `src/analysis/store/db/sqlite.rs:69` |
| `PreflopRowHash` | `struct(HashMap<String, PreflopRow>)` | `src/analysis/store/heads_up.rs:176` |
| `HUP` | `struct` | `src/analysis/store/heads_up.rs:210` |
| `Arrayable` | `trait` | `src/arrays/mod.rs:74` |
| `HandRanker` | `trait` | `src/arrays/mod.rs:79` |
| `BotError` | `enum` | `src/bot/profile.rs:124` |
| `CKCNumber` | `type(u32)` | `src/card_number.rs:4` |
| `CardsCell` | `struct(RefCell<Cards>)` | `src/cards_cell.rs:14` |
| `Stack` | `struct(Cell<usize>)` | `src/casino/cashier/chips.rs:9` |
| `Dealer` | `struct` | `src/casino/dealer.rs:182` |
| `Position` | `enum` | `src/casino/position.rs:13` |
| `SessionStep` | `enum` | `src/casino/session.rs:77` |
| `SessionState` | `struct` | `src/casino/session.rs:1000` |
| `Player` | `struct` | `src/casino/table/player.rs:24` |
| `Seat` | `struct` | `src/casino/table/seat.rs:23` |
| `Seats` | `struct(Vec<Seat>)` | `src/casino/table/seats.rs:26` |
| `BettingState` | `enum` | `src/casino/table/snapshot.rs:64` |
| `SeatState` | `struct` | `src/casino/table/snapshot.rs:113` |
| `TableState` | `struct` | `src/casino/table/snapshot.rs:143` |
| `California` | `struct` | `src/games/razz/california.rs:15` |
| `HandHistoryError` | `enum` | `src/hand_history.rs:890` |
| `AgentFidelity` | `struct` | `src/hand_history.rs:2375` |
| `PKError` | `enum` | `src/lib.rs:571` |
| `Betting` | `trait` | `src/lib.rs:802` |
| `Forgiving` | `trait` | `src/lib.rs:827` |
| `PhaseHoldemTracker` | `struct(Cell<PhaseHoldem>)` | `src/play/phases.rs:42` |
| `DealEval` | `struct` | `src/play/stages/deal_eval.rs:22` |
| `FlopEval` | `struct` | `src/play/stages/flop_eval.rs:199` |
| `RiverEval` | `struct` | `src/play/stages/river_eval.rs:36` |
| `TurnEval` | `struct` | `src/play/stages/turn_eval.rs:20` |
| `PokerBenchError` | `enum` | `src/pokerbench/error.rs:12` |
| `CanonicalSeating` | `struct` | `src/pokerbench/scenario.rs:134` |
| `Rank` | `enum` | `src/rank.rs:19` |
| `TestData` | `enum` | `src/util/data.rs:54` |
| `Util` | `struct` | `src/util/mod.rs:42` |
| `Terminal` | `struct` | `src/util/terminal.rs:28` |

## All types by file

### `src/analysis/case_eval.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `CaseEval` | `struct` | value object | 86 |

### `src/analysis/case_evals.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `CaseEvals` | `struct(Vec<CaseEval>)` | value object | 30 |

### `src/analysis/class.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HandRankClass` | `enum` | value object | 13 |

### `src/analysis/equity/result.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Method` | `enum` | value object | 9 |
| `PlayerEquity` | `struct` | value object | 29 |
| `EquityReport` | `struct` | value object | 67 |

### `src/analysis/equity/spec.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayerSpec` | `enum` | value object | 17 |
| `EquityOptions` | `struct` | value object | 60 |
| `EquityRequest` | `struct` | value object | 136 |

### `src/analysis/ev.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Ev` | `struct` | value object | 43 |

### `src/analysis/eval.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Eval` | `struct` | value object | 210 |
| `SevenEval` | `struct` | value object | 469 |

### `src/analysis/evals.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Evals` | `struct(Vec<Eval>)` | value object | 25 |

### `src/analysis/gto/combo.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Qualifier` | `enum` | value object | 16 |
| `Combo` | `struct` | value object | 43 |

### `src/analysis/gto/combo_pairs.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ComboPairs` | `struct(HashMap<Combo, Twos>)` | value object | 13 |

### `src/analysis/gto/combo_range.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ComboRangeForm` | `enum` | value object | 12 |
| `ComboRange` | `struct` | value object | 32 |

### `src/analysis/gto/combos.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Combos` | `struct(HashSet<Combo>)` | value object | 18 |

### `src/analysis/gto/game_tree.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `NodeId` | `struct(usize)` | value object | 74 |
| `Player` | `enum` | value object | 141 |
| `Action` | `enum` | value object | 194 |
| `TerminalOutcome` | `enum` | value object | 234 |
| `ActionNode` | `struct` | value object | 270 |
| `ChanceNode` | `struct` | value object | 299 |
| `TerminalNode` | `struct` | value object | 326 |
| `Node` | `enum` | value object | 358 |
| `GameTree` | `struct(Vec<Node>)` | cohesive mechanism | 447 |

### `src/analysis/gto/odds.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `WinLoseDraw` | `struct` | value object | 12 |

### `src/analysis/gto/regret.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `RegretAccumulator` | `struct(HashMap<NodeId, HashMap<Two, Vec<f64>>>)` | cohesive mechanism | 92 |

### `src/analysis/gto/solver.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SolverError` | `enum` | *(none)* | 117 |
| `SolverResult` | `struct` | value object | 206 |
| `Solver` | `struct` | cohesive mechanism | 577 |

### `src/analysis/gto/solver_cache.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SolverCache` | `struct(PathBuf)` | *(none)* | 190 |

### `src/analysis/gto/solver_config.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `CfrVariant` | `enum` | value object | 55 |
| `BetSize` | `struct` | value object | 116 |
| `BetSizings` | `struct` | value object | 318 |
| `SolverConfig` | `struct` | value object | 406 |

### `src/analysis/gto/strategy_profile.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ActionFrequencies` | `struct(Vec<f64>)` | value object | 69 |
| `StrategyProfile` | `struct(HashMap<NodeId, HashMap<Two, ActionFrequencies>>)` | cohesive mechanism | 274 |

### `src/analysis/gto/twos.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Twos` | `struct(HashSet<Two>)` | value object | 31 |

### `src/analysis/gto/vs.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Versus` | `struct` | *(none)* | 22 |

### `src/analysis/gto/weighted_combos.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `WeightedCombos` | `struct(HashMap<Combo, u8>)` | value object | 59 |

### `src/analysis/hand_rank.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HandRankValue` | `type(u16)` | value object | 20 |
| `HandRank` | `struct` | value object | 39 |

### `src/analysis/mod.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayOut` | `trait` | *(none)* | 32 |

### `src/analysis/name.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HandRankName` | `enum` | value object | 14 |

### `src/analysis/nubibus.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Fg` | `struct(T)` | *(none)* | 26 |
| `Cyan` | `struct` | *(none)* | 32 |
| `LightBlack` | `struct` | *(none)* | 33 |
| `Yellow` | `struct` | *(none)* | 34 |
| `LightBlue` | `struct` | *(none)* | 35 |
| `LightRed` | `struct` | *(none)* | 36 |
| `Green` | `struct` | *(none)* | 37 |
| `LightGreen` | `struct` | *(none)* | 38 |
| `Magenta` | `struct` | *(none)* | 39 |
| `Reset` | `struct` | *(none)* | 40 |
| `Nubificus` | `struct` | *(none)* | 48 |
| `Pluribus` | `struct` | *(none)* | 763 |
| `PluribusEvent` | `enum` | *(none)* | 1460 |

### `src/analysis/omaha.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `EightOrBetter` | `enum` | value object | 11 |

### `src/analysis/outs.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Outs` | `struct(IndexMap<usize, Cards>)` | value object | 14 |

### `src/analysis/player_stats.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayerStats` | `struct` | read model | 61 |
| `Confidence` | `enum` | value object | 232 |
| `StatsRegistry` | `struct` | *(none)* | 277 |

### `src/analysis/player_stats_store.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayerStatsStore` | `trait` | *(none)* | 63 |
| `YamlPlayerStatsStore` | `struct(PathBuf)` | *(none)* | 117 |

### `src/analysis/player_wins.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayerWins` | `struct(Wins)` | *(none)* | 14 |

### `src/analysis/pot_odds.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PotOdds` | `struct` | value object | 28 |

### `src/analysis/range_equity.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `RangeEquity` | `struct` | value object | 53 |

### `src/analysis/store/bcm/binary_card_map.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `FiveBCM` | `struct` | value object | 115 |
| `SevenFiveBCM` | `struct` | value object | 210 |

### `src/analysis/store/bcm/index_card_map.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `IndexCardMap` | `struct` | *(none)* | 43 |

### `src/analysis/store/db/hup.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HUPResult` | `struct` | value object | 30 |

### `src/analysis/store/db/sqlite.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Connect` | `struct(Connection)` | *(none)* | 3 |
| `Sqlable` | `trait` | *(none)* | 69 |

### `src/analysis/store/heads_up.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PreflopRow` | `struct` | value object | 101 |
| `PreflopRowHash` | `struct(HashMap<String, PreflopRow>)` | *(none)* | 176 |
| `HUP` | `struct` | *(none)* | 210 |

### `src/analysis/the_nuts.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `TheNuts` | `struct` | value object | 407 |

### `src/arrays/five/hands.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Hands` | `struct(Vec<Five>)` | value object | 50 |

### `src/arrays/five.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Five` | `struct([Card; 5])` | value object | 33 |

### `src/arrays/four.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Four` | `struct([Card; 4])` | value object | 27 |

### `src/arrays/hole_cards/twos.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `StartingHands` | `struct([Two; 9])` | value object | 35 |

### `src/arrays/matchups/masked.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Masked` | `struct` | value object | 86 |

### `src/arrays/matchups/masks/mod.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Mask` | `struct` | value object | 13 |
| `Masked` | `trait` | cohesive mechanism | 23 |

### `src/arrays/matchups/masks/rank_mask.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `RankMask` | `struct` | value object | 12 |

### `src/arrays/matchups/masks/suit_mask.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SuitMask` | `struct` | value object | 18 |

### `src/arrays/matchups/masks/suit_texture.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SuitTexture` | `enum` | value object | 14 |

### `src/arrays/matchups/shift.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Shifter` | `struct` | value object | 13 |

### `src/arrays/matchups/sorted_heads_up.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SortedHeadsUp` | `struct` | value object | 83 |

### `src/arrays/mod.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Arrayable` | `trait` | *(none)* | 74 |
| `HandRanker` | `trait` | *(none)* | 79 |

### `src/arrays/seven.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Seven` | `struct([Card; 7])` | value object | 24 |

### `src/arrays/six.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Six` | `struct([Card; 6])` | value object | 23 |

### `src/arrays/sliced.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `BoxedCards` | `struct(Box<[Card]>)` | value object | 30 |
| `Boxes` | `struct(Box<[BoxedCards]>)` | value object | 315 |

### `src/arrays/three.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Three` | `struct([Card; 3])` | value object | 20 |

### `src/arrays/two.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Two` | `struct([Card; 2])` | value object | 25 |

### `src/bard.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Bard` | `struct(u64)` | value object | 177 |

### `src/bot/betting_strategy.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Percentage` | `struct(u8)` | value object | 38 |
| `StreetAggression` | `struct` | value object | 198 |
| `BettingStrategy` | `struct` | value object | 236 |

### `src/bot/decider.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `BotDecider` | `trait` | cohesive mechanism | 77 |
| `RuleBasedDecider` | `struct` | cohesive mechanism | 160 |
| `JokerDecider` | `struct(Mutex<BotProfile>)` | cohesive mechanism | 387 |

### `src/bot/decision_config.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `DecisionConfig` | `struct` | value object | 31 |
| `EquityMode` | `enum` | value object | 69 |
| `RangeMode` | `enum` | value object | 90 |
| `PotOddsConfig` | `struct(f64)` | value object | 111 |
| `Toggle` | `enum` | value object | 129 |
| `ExploitMode` | `enum` | value object | 148 |
| `PreflopCharts` | `enum` | value object | 165 |

### `src/bot/exploit.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ExploitConfig` | `struct` | value object | 40 |

### `src/bot/exploitative_decider.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ExploitativeDecider` | `struct` | cohesive mechanism | 48 |

### `src/bot/playbook.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlaybookEntry` | `struct` | value object | 41 |
| `Playbook` | `struct(HashMap<u8, PlaybookEntry>)` | value object | 98 |

### `src/bot/position_ranges.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ActionRanges` | `struct(HashMap<String, WeightedRange>)` | value object | 41 |
| `PositionRanges` | `struct` | value object | 122 |

### `src/bot/positional_betting.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PositionalBetting` | `struct` | value object | 40 |

### `src/bot/profile.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayStyle` | `enum` | value object | 50 |
| `BotError` | `enum` | *(none)* | 124 |
| `BotProfile` | `struct` | value object | 210 |

### `src/bot/range_strategy.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `RangeStrategy` | `struct` | value object | 42 |

### `src/bot/sim.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ActionCounts` | `struct` | value object | 85 |
| `HandResult` | `struct` | value object | 145 |
| `SimResult` | `struct` | value object | 169 |
| `SimTable` | `struct` | cohesive mechanism | 214 |

### `src/bot/table_size.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `TableSize` | `enum` | value object | 34 |

### `src/bot/table_snapshot.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SeatInfo` | `struct` | read model | 54 |
| `TableSnapshot` | `struct` | read model | 116 |

### `src/bot/training/evaluator.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `FieldEntry` | `type((String, BotProfile))` | value object | 39 |

### `src/bot/training/trainer.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `TrainingConfig` | `struct` | value object | 43 |
| `GenerationRecord` | `struct` | value object | 97 |
| `TrainingResult` | `struct` | value object | 130 |
| `ExploitTrainer` | `struct` | cohesive mechanism | 167 |

### `src/bot/weighted_range.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ComboWeight` | `struct` | value object | 46 |
| `WeightedRange` | `struct(Vec<ComboWeight>)` | value object | 148 |

### `src/card.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Card` | `struct(u32)` | value object | 37 |

### `src/card_number.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `CKCNumber` | `type(u32)` | *(none)* | 4 |
| `CardNumber` | `enum` | value object | 73 |

### `src/cards.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Cards` | `struct(IndexSet<Card>)` | value object | 42 |

### `src/cards_cell.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `CardsCell` | `struct(RefCell<Cards>)` | *(none)* | 14 |

### `src/casino/action.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayerAction` | `enum` | action (DDD: command) | 45 |
| `TableAction` | `enum` | domain event | 96 |

### `src/casino/cashier/chips.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Stack` | `struct(Cell<usize>)` | *(none)* | 9 |

### `src/casino/dealer.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `DealerAction` | `enum` | action (DDD: command) | 53 |
| `DealerError` | `enum` | domain error | 97 |
| `Dealer` | `struct` | *(none)* | 182 |

### `src/casino/equity/seat_equity.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SeatEquity` | `struct` | value object | 15 |

### `src/casino/equity/seatbit.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Seatbit` | `struct(u16)` | value object | 13 |

### `src/casino/equity/table_equity.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `TableEquity` | `struct(Vec<SeatEquity>)` | value object | 14 |

### `src/casino/game.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ForcedBets` | `struct` | value object | 28 |

### `src/casino/position.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Position` | `enum` | *(none)* | 13 |
| `Positions` | `struct(Vec<Position>)` | value object | 101 |

### `src/casino/principal.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Principal` | `struct(Uuid)` | value object | 33 |

### `src/casino/session.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SessionStep` | `enum` | *(none)* | 77 |
| `PokerSession` | `struct` | consistency boundary (DDD: aggregate) | 125 |
| `SeatView` | `struct` | read model | 969 |
| `SessionState` | `struct` | *(none)* | 1000 |
| `SessionView` | `struct` | read model | 1042 |

### `src/casino/state.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PlayerState` | `enum` | value object | 21 |

### `src/casino/table/player.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Player` | `struct` | *(none)* | 24 |

### `src/casino/table/seat.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Seat` | `struct` | *(none)* | 23 |

### `src/casino/table/seats.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Seats` | `struct(Vec<Seat>)` | *(none)* | 26 |

### `src/casino/table/snapshot.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `BettingState` | `enum` | *(none)* | 64 |
| `SeatState` | `struct` | *(none)* | 113 |
| `TableState` | `struct` | *(none)* | 143 |

### `src/casino/table.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `VisibleHandMode` | `enum` | value object | 63 |
| `Table` | `struct` | consistency boundary (DDD: aggregate) | 102 |

### `src/casino/winnings.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Winnings` | `struct(Vec<PotWin>)` | value object | 11 |
| `PotWin` | `struct` | value object | 74 |

### `src/deck.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Deck` | `struct([Card; 52])` | value object | 20 |

### `src/games/betting_structure.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `BetTier` | `enum` | value object | 30 |
| `BettingStructure` | `enum` | value object | 58 |

### `src/games/kuhn.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `KuhnCard` | `enum` | value object | 56 |
| `KuhnAction` | `enum` | action (DDD: command) | 93 |
| `KuhnHistory` | `struct(Vec<KuhnAction>)` | value object | 138 |
| `KuhnInfoSet` | `struct` | read model | 265 |
| `KuhnState` | `struct` | consistency boundary (DDD: aggregate) | 330 |
| `KuhnStrategy` | `struct(HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>>)` | cohesive mechanism | 624 |
| `KuhnCfr` | `struct` | cohesive mechanism | 827 |

### `src/games/mod.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `GameFamily` | `enum` | value object | 34 |
| `GameType` | `enum` | value object | 123 |
| `GamePhase` | `enum` | value object | 266 |

### `src/games/omaha.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `OmahaHigh` | `struct(Four)` | value object | 37 |

### `src/games/razz/california.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `California` | `struct` | *(none)* | 15 |
| `CaliforniaHandRankValue` | `type(u16)` | value object | 67 |
| `CaliforniaHandRank` | `enum` | value object | 78 |

### `src/games/street.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `StreetIndex` | `struct(u8)` | value object | 39 |
| `StreetDescriptor` | `struct` | value object | 66 |

### `src/hand_history.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HandHistory` | `struct` | value object | 132 |
| `PlayerSnapshot` | `type((u8, String, usize, Option<String>, Option<Uuid>))` | value object | 202 |
| `HandHistoryError` | `enum` | *(none)* | 890 |
| `HandCollection` | `struct` | value object | 1005 |
| `HandMeta` | `struct` | value object | 1401 |
| `HandVariant` | `enum` | value object | 1438 |
| `TableInfo` | `struct` | value object | 1485 |
| `Stakes` | `struct` | value object | 1525 |
| `PlayerEntry` | `struct` | value object | 1574 |
| `PostedBlind` | `enum` | value object | 1741 |
| `Streets` | `struct` | value object | 1784 |
| `PreflopStreet` | `struct` | value object | 2027 |
| `FlopStreet` | `struct` | value object | 2055 |
| `TurnStreet` | `struct` | value object | 2110 |
| `RiverStreet` | `struct` | value object | 2161 |
| `AgentFidelity` | `struct` | *(none)* | 2375 |
| `Action` | `struct` | value object | 2436 |
| `ActionType` | `enum` | value object | 2486 |
| `ResultEntry` | `struct` | value object | 2537 |
| `Outcome` | `enum` | value object | 2614 |
| `AnalysisContext` | `struct` | value object | 2655 |
| `ReplayResult` | `struct` | value object | 2774 |

### `src/lib.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PKError` | `enum` | *(none)* | 571 |
| `Agency` | `trait` | specification | 788 |
| `Betting` | `trait` | *(none)* | 802 |
| `Forgiving` | `trait` | *(none)* | 827 |
| `GTO` | `trait` | cohesive mechanism | 870 |
| `Pile` | `trait` | value object | 890 |
| `Plurable` | `trait` | translator (DDD: anticorruption layer) | 1080 |
| `Unumable` | `trait` | translator (DDD: anticorruption layer) | 1102 |
| `SOK` | `trait` | specification | 1121 |
| `SuitShift` | `trait` | cohesive mechanism | 1130 |
| `Shifty` | `trait` | cohesive mechanism | 1148 |

### `src/play/board.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Board` | `struct` | value object | 22 |

### `src/play/game.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `StreetEquity` | `struct` | value object | 37 |
| `Game` | `struct` | value object | 272 |

### `src/play/hole_card.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HoleCard` | `struct` | value object | 34 |

### `src/play/hole_cards.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `HoleCards` | `struct(Vec<Two>)` | value object | 57 |

### `src/play/mod.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Position6Max` | `enum` | value object | 20 |

### `src/play/phases.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PhaseHoldem` | `enum` | value object | 13 |
| `PhaseHoldemTracker` | `struct(Cell<PhaseHoldem>)` | *(none)* | 42 |

### `src/play/seat_hand.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `SeatHand` | `struct` | value object | 49 |

### `src/play/stages/deal_eval.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `DealEval` | `struct` | *(none)* | 22 |

### `src/play/stages/flop_eval.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `FlopEval` | `struct` | *(none)* | 199 |

### `src/play/stages/river_eval.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `RiverEval` | `struct` | *(none)* | 36 |

### `src/play/stages/turn_eval.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `TurnEval` | `struct` | *(none)* | 20 |

### `src/play/visibility.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Visibility` | `enum` | value object | 32 |

### `src/pokerbench/action.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PokerBenchAction` | `enum` | value object | 31 |

### `src/pokerbench/error.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PokerBenchError` | `enum` | *(none)* | 12 |

### `src/pokerbench/scenario.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `PokerBenchSplit` | `enum` | value object | 37 |
| `PokerBenchScenario` | `struct` | value object | 88 |
| `CanonicalSeat` | `struct` | value object | 121 |
| `CanonicalSeating` | `struct` | *(none)* | 134 |

### `src/pokerbench/score.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `ActionScore` | `struct` | value object | 16 |

### `src/rank.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Rank` | `enum` | *(none)* | 19 |

### `src/ranks.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Ranks` | `struct(Vec<Rank>)` | value object | 11 |

### `src/suit.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Suit` | `enum` | value object | 15 |

### `src/util/data.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `TestData` | `enum` | *(none)* | 54 |

### `src/util/mod.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Percentage` | `struct` | value object | 21 |
| `Util` | `struct` | *(none)* | 42 |

### `src/util/random_ordering.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `RandomOrdering` | `struct(Ordering)` | cohesive mechanism | 11 |

### `src/util/terminal.rs`

| Type | Kind | Role | Line |
|------|------|------|-----:|
| `Terminal` | `struct` | *(none)* | 28 |

## Regenerate

```sh
/dddocs list
# or
python3 ~/.claude/skills/dddocs/scripts/check_dddocs.py --list .
```
