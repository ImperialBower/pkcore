/// Regression tests for full hand replays.
///
/// These mirror `examples/the_hand.rs` but run as integration tests so CI
/// catches regressions automatically. The hand is the 2006 High Stakes Poker
/// pot between Gus Hansen and Daniel Negreanu — eight seats, four streets, and
/// a showdown that turns on quad fives.
///
/// EPIC-83 retired the interior-mutable `TableCelled` twin these tests used to
/// run in parallel. Only `Table` remains, and it is the engine that deals to
/// the button's left — the rule the celled engine got wrong.
#[allow(nonstandard_style)]
mod hands__the_hand_tests {
    use pkcore::casino::action::TableAction;
    use pkcore::casino::winnings::Winnings;
    use pkcore::prelude::*;

    /// Drives the table through the complete hand and returns it ready for
    /// assertions.
    fn run_the_hand() -> Result<(Table, Winnings), PKError> {
        let mut table = TestData::the_hand_table();

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

        let winnings = table.end_hand()?;

        Ok((table, winnings))
    }

    /// The full hand must complete and `end_hand` must succeed.
    #[test]
    fn the_hand_completes_on_the_plain_table() {
        let result = run_the_hand();
        assert!(result.is_ok(), "the_hand failed: {:?}", result.err());
    }

    /// Gus Hansen (seat 3, 5♦ 5♣) makes quad fives on 9♣ 6♦ 5♥ 5♠ 8♠ and beats
    /// Daniel Negreanu's sixes full.
    ///
    /// This is the assertion EPIC-83 was missing: `examples/the_hand_no_cell.rs`
    /// walked the hand on `Table` but asserted nothing, so a wrong winner went
    /// unnoticed until the two engines were compared side by side.
    #[test]
    fn the_hand_gus_wins_on_the_plain_table() {
        let (table, winnings) = run_the_hand().expect("hand should complete");

        println!("\n=== Event Log ===");
        for action in &table.event_log {
            println!("{action}");
        }

        assert!(!winnings.is_empty(), "winnings should not be empty");

        // The exact event kind depends on whether the hand routed through
        // symmetric heads-up (`PlayerWins`) or the side-pot-aware multiway path
        // (`PlayerWinsMainPot` / `PlayerWinsSidePot`). Mismatched all-ins route
        // to multiway, so accept any of the three.
        let gus_won = table.event_log.iter().any(|e| match e {
            TableAction::PlayerWins(seat, _, _, _, _)
            | TableAction::PlayerWinsMainPot(seat, _)
            | TableAction::PlayerWinsSidePot(seat, _) => *seat == 3,
            _ => false,
        });
        assert!(gus_won, "expected a win event for seat 3 (Gus Hansen)");
    }

    /// The event log must contain entries for every street, confirming the full
    /// hand was replayed and not cut short.
    #[test]
    fn the_hand_event_log_contains_all_streets() {
        let (table, _) = run_the_hand().expect("hand should complete");

        let has_flop = table.event_log.iter().any(|e| matches!(e, TableAction::DealtFlop(_)));
        let has_turn = table.event_log.iter().any(|e| matches!(e, TableAction::DealtTurn(_)));
        let has_river = table.event_log.iter().any(|e| matches!(e, TableAction::DealtRiver(_)));

        assert!(has_flop, "event log missing DealtFlop");
        assert!(has_turn, "event log missing DealtTurn");
        assert!(has_river, "event log missing DealtRiver");
    }
}
