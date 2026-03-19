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
    fn deals_to_river_after_preflop_all_ins__poor_man_then_rich() {
        let table = preroll(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣",
        );

        println!("{table}");

        let hand_result = table.end_hand().expect("hand should end successfully");
        println!("{hand_result}");
    }

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
}
