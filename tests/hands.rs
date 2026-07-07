/// Regression tests for full hand replays.
///
/// These tests mirror the `examples/the_hand.rs` example but run as integration
/// tests so CI catches regressions automatically.  The key regression being
/// protected here is the recursive borrow bug that was fixed in `showdown.rs`:
/// `effective_player_cards()` and `log_info(PlayerWins)` both call `get_seat()`
/// internally; previously the mutable `RefCell` borrow of the winning seat was
/// still held when those helpers were invoked, causing a panic at runtime.
#[allow(nonstandard_style)]
mod hands__the_hand_tests {
    use pkcore::casino::action::TableAction;
    use pkcore::casino::winnings::Winnings;
    use pkcore::prelude::*;

    /// Drives the table through the complete hand and returns it ready for
    /// assertions.  Mirrors the helper functions in `examples/the_hand.rs`.
    fn run_the_hand() -> Result<(TableCelled, Winnings), PKError> {
        let table = TestData::the_hand_table();

        // ── setup ──────────────────────────────────────────────────────────────
        table.act_forced_bets()?;
        table.deal_cards_to_seats()?;

        // ── preflop ────────────────────────────────────────────────────────────
        table.act_bet(3, 2100)?;
        table.act_raise(4, 5000)?;
        table.act_fold(5)?;
        table.act_fold(6)?;
        table.act_fold(7)?;
        table.act_fold(0)?;
        table.act_fold(1)?;
        table.act_fold(2)?;
        table.act_call(3)?;
        table.bring_it_in()?;

        // ── flop ───────────────────────────────────────────────────────────────
        table.deal_flop()?;
        table.act_check(3)?;
        table.act_bet(4, 8_000)?;
        table.act_raise(3, 26_000)?;
        table.act_call(4)?;
        table.bring_it_in()?;

        // ── turn ───────────────────────────────────────────────────────────────
        table.deal_turn()?;
        table.act_bet(3, 24_000)?;
        table.act_call(4)?;
        table.bring_it_in()?;

        // ── river ──────────────────────────────────────────────────────────────
        table.deal_river()?;
        table.act_check(3)?;
        table.act_bet(4, 65_000)?;
        table.act_all_in(3)?;
        table.act_call(4)?;

        // This is where the recursive borrow bug previously caused a panic.
        let winnings = table.end_hand()?;

        Ok((table, winnings))
    }

    /// The full hand must complete without panicking and `end_hand` must succeed.
    /// This is the primary regression guard for the recursive-borrow fix in
    /// `showdown.rs`.
    #[test]
    fn test_the_hand_completes_without_panic() {
        let result = run_the_hand();
        assert!(result.is_ok(), "the_hand failed: {:?}", result.err());
    }

    /// Gus Hansen (seat 3, 5♦ 5♣) hits four-of-a-kind fives on the board
    /// (9♣ 6♦ 5♥ 5♠ 8♠) and wins the pot.  The event log must contain a
    /// `PlayerWins` entry for seat 3.
    #[test]
    fn test_the_hand_gus_wins() {
        let (table, winnings) = run_the_hand().expect("hand should complete");

        // Dump the event log so failures are easy to diagnose.
        println!("\n=== Event Log ===\n{}", table.event_log);

        assert!(!winnings.is_empty(), "winnings should not be empty");

        // At least one win event must record seat 3 (Gus Hansen). The exact
        // event kind depends on whether the hand routed through symmetric
        // heads-up (`PlayerWins`) or the side-pot-aware multiway path
        // (`PlayerWinsMainPot` / `PlayerWinsSidePot`). Mismatched all-ins
        // route to multiway, so accept any of the three.
        let gus_won = table.event_log.entries().iter().any(|e| match e {
            TableAction::PlayerWins(seat, _, _, _, _)
            | TableAction::PlayerWinsMainPot(seat, _)
            | TableAction::PlayerWinsSidePot(seat, _) => *seat == 3,
            _ => false,
        });
        assert!(gus_won, "expected a win event for seat 3 (Gus Hansen)");
    }

    /// The event log must contain entries for every street (deal, flop, turn,
    /// river) to confirm the full hand was replayed correctly.
    #[test]
    fn test_the_hand_event_log_contains_all_streets() {
        let (table, _) = run_the_hand().expect("hand should complete");

        println!("\n=== Event Log ===\n{}", table.event_log);

        let entries = table.event_log.entries();

        let has_flop = entries.iter().any(|e| matches!(e, TableAction::DealtFlop(_)));
        let has_turn = entries.iter().any(|e| matches!(e, TableAction::DealtTurn(_)));
        let has_river = entries.iter().any(|e| matches!(e, TableAction::DealtRiver(_)));

        assert!(has_flop, "event log missing DealtFlop");
        assert!(has_turn, "event log missing DealtTurn");
        assert!(has_river, "event log missing DealtRiver");
    }
}
