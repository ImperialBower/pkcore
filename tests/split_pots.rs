#[allow(nonstandard_style)]
mod casino__table_split_pot_tests {
    use pkcore::prelude::*;

    fn preroll(index: &str) -> TableCelled {
        let table = TestData::split_pot_table(&cc!(index));

        assert_eq!(46, table.deck.len());

        table.act_forced_bets().expect("forced bets should post");

        assert!(table.seats.are_dealt());
        assert_eq!(3, table.seats.count_players_with_action_to_give());

        table.act_all_in(0).expect("seat 0 should be able to go all-in");
        table.act_all_in(1).expect("seat 1 should be able to go all-in");
        table.act_all_in(2).expect("seat 2 should be able to go all-in");

        assert_eq!(PlayerState::Bet(9_000), table.get_seat(0).unwrap().player.state.get());

        assert_eq!(1, table.seats.count_players_with_action_to_give());
        assert!(table.is_betting_complete());

        table.bring_it_in().expect("flop should be dealt");

        assert_eq!(PlayerState::Bet(9_000), table.get_seat(0).unwrap().player.state.get());

        table.deal_flop().expect("flop should be dealt");

        assert_eq!(PlayerState::Bet(9_000), table.get_seat(0).unwrap().player.state.get());
        assert_eq!(1, table.seats.count_players_with_action_to_give());
        assert!(!table.is_game_over());
        assert!(table.is_betting_complete());

        table.deal_turn().expect("turn should be dealt");
        assert_eq!(PlayerState::Bet(9_000), table.get_seat(0).unwrap().player.state.get());
        assert!(!table.is_game_over());
        assert!(table.is_betting_complete());

        table.deal_river().expect("river should be dealt");
        assert_eq!(PlayerState::Bet(9_000), table.get_seat(0).unwrap().player.state.get());
        assert!(table.is_betting_complete());
        assert!(table.is_game_over());

        table
    }

    /// Winning scenarios:
    ///
    /// -
    #[test]
    fn deals_to_river_after_preflop_all_ins__rich_man() {
        let table = preroll(
            "K♠ Q♠ Q♥ Q♣ J♠ A♦ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );

        let winnings = table.end_hand().expect("hand should end successfully");
        assert!(
            !winnings.is_empty(),
            "winnings should not be empty after a completed hand"
        );
    }

    #[test]
    fn deals_to_river_after_preflop_all_ins__average() {
        let table = preroll(
            "4♠ Q♠ 8♠ 4♥ J♠ A♣ A♦ T♠ K♠ 9♠ 7♠ 6♠ 5♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );

        let winnings = table.end_hand().expect("hand should end successfully");
        assert!(
            !winnings.is_empty(),
            "winnings should not be empty after a completed hand"
        );
    }

    #[test]
    fn deals_to_river_after_preflop_all_ins__poor_man_then_rich() {
        let table = preroll(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );

        // Verify chips in play for each seat before resolving the hand
        let s0_chips_in_play = table.get_seat(0).unwrap().player.get_chips_in_play();
        let s1_chips_in_play = table.get_seat(1).unwrap().player.get_chips_in_play();
        let s2_chips_in_play = table.get_seat(2).unwrap().player.get_chips_in_play();

        // Expectations derived from the simulated actions in `preroll`:
        // - Seat 0 (Rich Man) bet 9_000 into play
        // - Seat 1 (Poor Man) went all-in with 5_000
        // - Seat 2 (Average Person) went all-in with 9_000
        assert_eq!(s0_chips_in_play, 9_000, "Seat 0 chips_in_play");
        assert_eq!(s1_chips_in_play, 5_000, "Seat 1 chips_in_play");
        assert_eq!(s2_chips_in_play, 9_000, "Seat 2 chips_in_play");

        let winnings = table.end_hand().expect("hand should end successfully");

        assert!(!table.event_log.entries().is_empty(), "event log should have entries");
        // Two separate pot wins: Poor Man takes the main pot, Rich Man takes the side pot.
        assert!(winnings.len() >= 2, "split pot should produce at least two pot wins");
    }

    #[test]
    fn plus_blinds() {
        let table = TestData::preroll_split_pot_with_blinds__to_completion(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 6♣ 5♣ 3♣ 2♣",
        );

        // Two equal stacks (seats 0 and 4 at 9_000), one short stack (seat 3 at 5_000),
        // and two separate folded-blind NONE entries (50 and 100) — four equity groups.
        // NONE entries are intentionally kept separate so winnings() counts each
        // as an independent contributor rather than a single combined pool.
        let equity = table.determine_hand_equity();
        assert_eq!(4, equity.len(), "expected four equity groups");
        assert_eq!(9_000, equity.ceiling(), "ceiling should be the short-stack threshold");

        let winnings = table.end_hand().expect("hand should end successfully");
        assert!(
            !winnings.is_empty(),
            "winnings should not be empty after a completed hand"
        );
    }
}

#[allow(nonstandard_style)]
mod casino__table_no_cell__split_pot_tests {
    use pkcore::arrays::sliced::BoxedCards;
    use pkcore::cards::Cards;
    use pkcore::casino::game::ForcedBets;
    use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    use std::str::FromStr;

    /// Regression test: BB folds after posting 100 when all others go all-in for < 100.
    ///
    /// Trigger: BB's `chips_in_play` (100) exceeds every active (non-folded) player's
    /// `chips_in_play` (max = 80).  Before the fix, `showdown_multiway()` would break
    /// out of its side-pot loop with `eligible_seats.is_empty()`, leaving the orphaned
    /// `Seatbit::NONE` chips undistributed and causing `end_hand()` to return
    /// `Err(PKError::ChipAuditFailed)`.
    ///
    /// After the fix, orphaned NONE chips are drained to the most recent pot winner so
    /// chip conservation holds and `end_hand()` returns `Ok(winnings)`.
    #[test]
    fn bb_folds_over_contribution_no_chip_loss() {
        // Pre-set hole cards on seats so nlh_from_seats() removes them from the deck,
        // preventing duplicates in community cards.
        let mut seat0 = SeatNoCell::new(PlayerNoCell::new_with_chips("BTN".to_string(), 70));
        seat0.cards = BoxedCards::from_str("7♦ 2♣").unwrap();
        let mut seat1 = SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 80));
        seat1.cards = BoxedCards::from_str("8♦ 3♣").unwrap();
        let mut seat2 = SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 600));
        seat2.cards = BoxedCards::from_str("9♠ 4♦").unwrap();
        let mut seat3 = SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 30));
        seat3.cards = BoxedCards::from_str("A♠ A♥").unwrap();

        let seats = SeatsNoCell::new(vec![seat0, seat1, seat2, seat3]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // SB posts 50, BB posts 100; hand_chip_total = 280.
        table.act_forced_bets().unwrap();

        // Pre-flop action: UTG → BTN → SB → BB
        let utg = table.next_to_act(); // seat 3: 30 chips_in_play
        table.act_all_in(utg).unwrap();
        let btn = table.next_to_act(); // seat 0: 70 chips_in_play
        table.act_all_in(btn).unwrap();
        let sb = table.next_to_act(); // seat 1: 80 chips_in_play
        table.act_all_in(sb).unwrap();
        let bb = table.next_to_act(); // seat 2: no full raise; BB folds
        table.act_fold(bb).unwrap(); // chips_in_play=100 > max_active=80 → orphaned NONE entry

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(
            result.is_ok(),
            "end_hand should not fail with ChipAuditFailed when BB over-contributes and folds: {result:?}"
        );
    }

    /// Replaces `table.deck` with a deck whose first 8 cards drive the burn / flop /
    /// turn / river deal in deterministic order, and whose remaining 40 cards are
    /// any cards not already used as hole cards or in the rigged top.
    ///
    /// The pre-set hole cards must already have been removed from the table's deck
    /// by `nlh_from_seats`; this helper just re-orders so that draws are
    /// deterministic.
    fn rig_deck(table: &mut TableNoCell, top: &str, hole_and_top: &str) {
        let used = Cards::from_str(hole_and_top).expect("used cards parse");
        let mut deck = Cards::from_str(top).expect("top cards parse");
        let rest = Cards::deck_minus(&used);
        deck.insert_all(&rest);
        table.deck = deck;
    }

    /// Heads-up showdown with mismatched all-ins and a tied result.
    ///
    /// Seat 0 (deep, 1000) and seat 1 (short, 200) both go all-in pre-flop.
    /// A rigged board (`A♥ A♦ A♣ A♠ K♥`) gives both players four-aces with a
    /// king kicker — playing the board, identical hand strength, exact tie.
    ///
    /// Correct payout (per side-pot semantics):
    /// - Main pot capped at the short stack (200 from each player) = 400, split
    ///   50/50 → 200 each
    /// - Uncalled portion (800 chips of seat 0's bet that nobody could match)
    ///   returned to seat 0
    /// - Final stacks: seat 0 = 1000 (no change), seat 1 = 200 (no change)
    ///
    /// Defect (pre-fix) `showdown_headsup` simply does
    /// `divvy_up(self.pot, winners.len())` = `divvy_up(1200, 2) = [600, 600]`,
    /// giving seat 0 only 600 (a 400-chip loss on a tied hand) and seat 1 600
    /// (a 400-chip win on a tied hand). The chips are conserved but allocated
    /// to the wrong winner.
    #[test]
    fn heads_up_tied_with_short_all_in_returns_uncalled_excess() {
        let mut seat0 = SeatNoCell::new(PlayerNoCell::new_with_chips("Deep".to_string(), 1_000));
        seat0.cards = BoxedCards::from_str("7♦ 2♣").unwrap();
        let mut seat1 = SeatNoCell::new(PlayerNoCell::new_with_chips("Short".to_string(), 200));
        seat1.cards = BoxedCards::from_str("4♦ 5♦").unwrap();

        let seats = SeatsNoCell::new(vec![seat0, seat1]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // Burn-flop-burn-turn-burn-river: 6♣, [A♥ A♦ A♣], 6♦, A♠, 6♥, K♥.
        // Both players play four aces from the board → exact tie.
        rig_deck(
            &mut table,
            "6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
            "7♦ 2♣ 4♦ 5♦ 6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
        );

        // Heads-up: button (seat 0) is SB, seat 1 is BB. SB acts first preflop.
        table.act_forced_bets().unwrap();
        let utg = table.next_to_act(); // seat 0 (SB)
        table.act_all_in(utg).unwrap();
        let next = table.next_to_act(); // seat 1 (BB)
        table.act_all_in(next).unwrap();

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        let s0 = table.seats.get_seat(0).unwrap().player.chips;
        let s1 = table.seats.get_seat(1).unwrap().player.chips;
        assert_eq!(
            1_200,
            table.table_chip_count(),
            "chips must be conserved across the hand"
        );
        assert_eq!(
            1_000, s0,
            "tied heads-up: deep stack must end at starting stack (200 won from main + 800 uncalled returned), got {s0}"
        );
        assert_eq!(
            200, s1,
            "tied heads-up: short stack must end at starting stack (200 won from main, no eligibility for uncalled), got {s1}"
        );
    }

    /// Heads-up showdown where the short stack wins outright. The deep stack's
    /// uncalled excess (the chips nobody could match) must be returned, not
    /// awarded to the short winner.
    ///
    /// Seat 0 (deep, 1000) is dealt 7♦ 2♣ → garbage. Seat 1 (short, 200) is
    /// dealt A♠ A♥ → premium. Rigged board K♠ K♣ 9♦ 8♠ 4♥ leaves seat 1 with
    /// AAKK9 (two pair, aces and kings) and seat 0 with KK987 (one pair). Seat 1
    /// wins.
    ///
    /// Correct payout: seat 1 wins the main pot (400 = 200 from each), seat 0
    /// gets back the 800 uncalled excess. Final: seat 0 = 800, seat 1 = 400.
    ///
    /// Defect (pre-fix): `divvy_up(1200, 1) = [1200]` awards the entire pot to
    /// the short winner — seat 1 ends with 1200 (a 1000-chip win on a 200-chip
    /// stack), seat 0 ends with 0 (their uncalled 800 absorbed by the winner).
    #[test]
    fn heads_up_short_winner_excess_returned_to_deep_stack() {
        let mut seat0 = SeatNoCell::new(PlayerNoCell::new_with_chips("Deep".to_string(), 1_000));
        seat0.cards = BoxedCards::from_str("7♦ 2♣").unwrap();
        let mut seat1 = SeatNoCell::new(PlayerNoCell::new_with_chips("Short".to_string(), 200));
        seat1.cards = BoxedCards::from_str("A♠ A♥").unwrap();

        let seats = SeatsNoCell::new(vec![seat0, seat1]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // Burn-flop-burn-turn-burn-river: 3♣, [K♠ K♣ 9♦], 3♥, 8♠, 3♦, 4♥.
        // Seat 1 (AA) makes two pair AAKK with 9 kicker.
        // Seat 0 (72) makes one pair KK with 9-8-7 kickers. Seat 1 wins.
        rig_deck(
            &mut table,
            "3♣ K♠ K♣ 9♦ 3♥ 8♠ 3♦ 4♥",
            "7♦ 2♣ A♠ A♥ 3♣ K♠ K♣ 9♦ 3♥ 8♠ 3♦ 4♥",
        );

        table.act_forced_bets().unwrap();
        let utg = table.next_to_act();
        table.act_all_in(utg).unwrap();
        let next = table.next_to_act();
        table.act_all_in(next).unwrap();

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        let s0 = table.seats.get_seat(0).unwrap().player.chips;
        let s1 = table.seats.get_seat(1).unwrap().player.chips;
        assert_eq!(
            1_200,
            table.table_chip_count(),
            "chips must be conserved across the hand"
        );
        assert_eq!(
            800, s0,
            "deep stack lost main pot but must reclaim uncalled 800, got {s0}"
        );
        assert_eq!(
            400, s1,
            "short winner only eligible for matched 200+200 main pot, got {s1}"
        );
    }

    /// Regression guard: heads-up with **equal** stacks and tied hands must
    /// produce an even split via the existing fast path. This protects against
    /// over-correcting the asymmetric fix and breaking the symmetric case.
    #[test]
    fn heads_up_symmetric_tied_split_50_50() {
        let mut seat0 = SeatNoCell::new(PlayerNoCell::new_with_chips("Equal0".to_string(), 1_000));
        seat0.cards = BoxedCards::from_str("7♦ 2♣").unwrap();
        let mut seat1 = SeatNoCell::new(PlayerNoCell::new_with_chips("Equal1".to_string(), 1_000));
        seat1.cards = BoxedCards::from_str("4♦ 5♦").unwrap();

        let seats = SeatsNoCell::new(vec![seat0, seat1]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // Same four-aces-on-board rig as the asymmetric tied test → exact tie.
        rig_deck(
            &mut table,
            "6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
            "7♦ 2♣ 4♦ 5♦ 6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
        );

        table.act_forced_bets().unwrap();
        let utg = table.next_to_act();
        table.act_all_in(utg).unwrap();
        let next = table.next_to_act();
        table.act_all_in(next).unwrap();

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        let s0 = table.seats.get_seat(0).unwrap().player.chips;
        let s1 = table.seats.get_seat(1).unwrap().player.chips;
        assert_eq!(
            2_000,
            table.table_chip_count(),
            "chips must be conserved across the hand"
        );
        assert_eq!(
            1_000, s0,
            "symmetric tied heads-up: each gets back their stack, got s0={s0}"
        );
        assert_eq!(
            1_000, s1,
            "symmetric tied heads-up: each gets back their stack, got s1={s1}"
        );
    }

    /// Three-way all-in with **asymmetric** chip commitments and all three
    /// players tied at showdown. This exercises a `showdown_multiway` edge
    /// case: when overall winners have different commitments, processing
    /// must distribute each pot layer in turn to all winners eligible for
    /// it. Layer 1 (cap 100) splits 3 ways; layer 2 (cap 100→200) splits 2
    /// ways between the two deeper winners; layer 3 (cap 200→500) goes
    /// solo to the deepest winner. Expected: every player ends at their
    /// starting stack (chop).
    ///
    /// Buggy form (`processed_chip_levels` keying on raw chip count): the
    /// second-layer iteration sees `chip_level == 100` after subtraction
    /// and incorrectly matches the first iteration's processed value,
    /// skipping layer 2 distribution and absorbing those chips into a
    /// later iteration. The medium stack ends short and the deepest stack
    /// ends long.
    #[test]
    fn three_way_asymmetric_tied_chops_correctly() {
        let mut seat_a = SeatNoCell::new(PlayerNoCell::new_with_chips("Short".to_string(), 100));
        seat_a.cards = BoxedCards::from_str("7♦ 2♣").unwrap();
        let mut seat_b = SeatNoCell::new(PlayerNoCell::new_with_chips("Mid".to_string(), 200));
        seat_b.cards = BoxedCards::from_str("4♦ 5♦").unwrap();
        let mut seat_c = SeatNoCell::new(PlayerNoCell::new_with_chips("Deep".to_string(), 500));
        seat_c.cards = BoxedCards::from_str("8♥ 9♥").unwrap();

        let seats = SeatsNoCell::new(vec![seat_a, seat_b, seat_c]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // Same four-aces-on-board rig — all three players play the board.
        rig_deck(
            &mut table,
            "6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
            "7♦ 2♣ 4♦ 5♦ 8♥ 9♥ 6♣ A♥ A♦ A♣ 6♦ A♠ 6♥ K♥",
        );

        table.act_forced_bets().unwrap();
        // 3-handed: button=0 → SB=1, BB=2; UTG (first to act preflop) = seat 0.
        let utg = table.next_to_act();
        table.act_all_in(utg).unwrap();
        let next = table.next_to_act();
        table.act_all_in(next).unwrap();
        let next = table.next_to_act();
        table.act_all_in(next).unwrap();

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        let s0 = table.seats.get_seat(0).unwrap().player.chips;
        let s1 = table.seats.get_seat(1).unwrap().player.chips;
        let s2 = table.seats.get_seat(2).unwrap().player.chips;
        assert_eq!(800, table.table_chip_count(), "chips must be conserved across the hand");
        assert_eq!(100, s0, "short tied chop: must end at starting stack 100, got {s0}");
        assert_eq!(200, s1, "mid tied chop: must end at starting stack 200, got {s1}");
        assert_eq!(500, s2, "deep tied chop: must end at starting stack 500, got {s2}");
    }
}
