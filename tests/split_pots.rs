#[allow(nonstandard_style)]
mod casino__table_split_pot_tests {
    use pkcore::prelude::*;

    fn preroll(index: &str) -> Table {
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

        println!("{table}");

        let hand_result = table.end_hand().expect("hand should end successfully");
        println!("{hand_result}");
    }

    #[test]
    fn deals_to_river_after_preflop_all_ins__average() {
        let table = preroll(
            "4♠ Q♠ 8♠ 4♥ J♠ A♣ A♦ T♠ K♠ 9♠ 7♠ 6♠ 5♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );

        println!("{table}");

        let hand_result = table.end_hand().expect("hand should end successfully");
        println!("{hand_result}");
    }

    #[test]
    fn deals_to_river_after_preflop_all_ins__poor_man_then_rich() {
        let table = preroll(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );
        println!("{table}");

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

        let hand_result = table.end_hand().expect("hand should end successfully");

        println!("{}", table.event_log);

        println!("{hand_result}");
    }

    #[test]
    fn plus_blinds() {
        // let cards = cc!(
        //     "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 6♣ 5♣ 3♣ 2♣"
        // );

        // let table = TestData::split_pot_table_with_blinds(&cards);
        let table = TestData::preroll_split_pot_with_blinds__to_completion(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 6♣ 5♣ 3♣ 2♣",
        );

        println!("{}", table.determine_hand_equity());

        table.end_hand2().expect("hand should end successfully");
    }
}
