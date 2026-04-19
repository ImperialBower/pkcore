# Code Review: Uncommitted Bug Fix (`showdown.rs`)

**Scope:** Uncommitted changes fixing orphaned dead-money chips in `process_multiway()`.

## Summary of Changes
- Added a `last_winner` tracker in `showdown_multiway` (`src/casino/table/showdown.rs`) to track the previous pot winner.
- Replaced a silent loop `break` with logic to correctly distribute orphaned dead-money chips to the `last_winner`.
- Added tests `process_multiway__bb_folds_over_contribution_no_chip_loss` to enforce chip conservation, and `process_multiway__bb_folds_over_contribution_winnings_non_empty` for the `Winnings` result.
- Added data generation utility methods in `src/util/data.rs` to create a `TableCelled` with pre-defined scenarios mimicking a large uncalled Big Blind contribution.

## Strengths
- **Correctness & Safety**: The fix beautifully corrects a silent failure state (losing orphaned chips when the BB over-contributes and then folds). By tracking `last_winner` and sweeping the remaining dead money to it (`Seatbit::NONE`), chip conservation math is strictly preserved.
- **Improved Test Fidelity**: `TestData::bb_folds_over_contribution_table` builds exactly the terminal state required to trigger the bug. The assertions in the tests not only cover the exact dropped value (`780usize.saturating_sub(total)`) but also verify `Winnings` validity.
- **Cross-Porting Alignment**: It directly mirrors the corresponding `table_no_cell.rs` fix (documented in `RCA_Table_Mechanic_2026.md`) addressing the exact same problem for the `TableCelled` path, ensuring correctness across both architectures.

## Areas for Improvement (Nitpicks)
- Setting `last_winner = Some(winner_with_lowest)` at the end of Phase 2 is technically redundant since Phase 2 will eventually break shortly after, but it's completely safe, defensively programmed, and keeps the semantic meaning intact.
- **Type Conversions**: The loop iterating over `0u8..16u8` could be refactored to use `Seatbit::CAPACITY` replacing `16u8` as detailed in `RCA_Table_Mechanic_2026.md`. This prevents magic number discrepancies.

## Verdict
**Approved**. The patch safely handles an unhandled terminal state and includes necessary regression testing to preserve mathematical correctness in the game engine.

