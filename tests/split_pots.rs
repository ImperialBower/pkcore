#[allow(nonstandard_style)]
mod casino__table_split_pot_tests {
    use pkcore::prelude::*;

    fn my_table(cards: CardsCell) -> Table {
        let rich = Seat {
            player: Player::new_with_chips("Rich Man".to_string(), 10_000),
            cards: boxed!("Q♦ Q♣"),
        };
        let poor = Seat {
            player: Player::new_with_chips("Poor Man".to_string(), 5_000),
            cards: boxed!("A♠ A♥"),
        };
        let average = Seat {
            player: Player::new_with_chips("Average Person".to_string(), 9_000),
            cards: boxed!("4♣ 4♦"),
        };
        let seats = Seats::new(vec![rich, poor, average]);

        Table::nlh_primed(seats, &cards, ForcedBets::new(50, 100))
    }

    #[test]
    fn deals_to_river_after_preflop_all_ins() {
        let table = my_table(cc!(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣"
        ));

        println!("{table}");
        println!("{}", table.deck);

        assert_eq!(46, table.deck.len());

        table.act_forced_bets().expect("forced bets should post");

        println!("{table}");

        assert!(table.seats.are_dealt());

        table.act_all_in(0).expect("seat 0 should be able to go all-in");
        table.act_all_in(1).expect("seat 1 should be able to go all-in");
        table.act_all_in(2).expect("seat 2 should be able to go all-in");

        println!("{table}");

        table.bring_it_in().expect("flop should be dealt");
        table.deal_flop().expect("flop should be dealt");
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
