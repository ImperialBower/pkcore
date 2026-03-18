#[allow(nonstandard_style)]
mod casino__table_split_pot_tests {
    use pkcore::prelude::*;

    #[test]
    fn deals_to_river_after_preflop_all_ins() {
        let table = TestData::split_pot_table(&cc!(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣"
        ));

        assert_eq!(46, table.deck.len());

        table.act_forced_bets().expect("forced bets should post");

        assert!(table.seats.are_dealt());
        assert_eq!(3, table.seats.count_players_with_action_to_give());

        table.act_all_in(0).expect("seat 0 should be able to go all-in");
        table.act_all_in(1).expect("seat 1 should be able to go all-in");
        table.act_all_in(2).expect("seat 2 should be able to go all-in");

        assert_eq!(1, table.seats.count_players_with_action_to_give());
        assert!(table.is_betting_complete());

        table.bring_it_in().expect("flop should be dealt");
        table.deal_flop().expect("flop should be dealt");

        assert_eq!(1, table.seats.count_players_with_action_to_give());
        assert!(!table.is_game_over());
        assert!(table.is_betting_complete());

        table.deal_turn().expect("turn should be dealt");
        assert!(!table.is_game_over());
        assert!(table.is_betting_complete());

        table.deal_river().expect("river should be dealt");
        assert!(table.is_betting_complete());
        assert!(table.is_game_over());

        println!("{table}");

        let hand_result = table.end_hand().expect("hand should end successfully");
        println!("{hand_result}");

        // assert!(table.is_flop());
        //
        // table.deal_turn().expect("turn should be dealt");
        // assert!(table.is_turn());
        //
        // table.deal_river().expect("river should be dealt");
        // assert!(table.is_river());
        // assert_eq!(5, table.board.len());
        //
        // println!("{table}");
        //
        // assert_eq!(0, table.seats.count_able_to_bet_in_hand());
        // assert!(table.seats.is_everyone_allin_except_one());
    }
}
