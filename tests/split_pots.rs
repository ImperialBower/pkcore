#[allow(nonstandard_style)]
mod casino__table_split_pot_tests {
    use pkcore::prelude::*;

    #[test]
    fn deals_to_river_after_preflop_all_ins() {
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
        let cards = seats.cards_snapshot().cards();

        let table = Table::nlh_primed(
            seats,
            &CardsCell::from(Cards::deck_primed(&cards)),
            ForcedBets::new(50, 100),
        );

        println!("{table}");

        table.act_forced_bets().expect("forced bets should post");

        println!("{table}");

        assert!(table.seats.are_dealt());

        table.act_all_in(0).expect("seat 0 should be able to go all-in");
        table.act_all_in(1).expect("seat 1 should be able to go all-in");
        table.act_all_in(2).expect("seat 2 should be able to go all-in");

        table.deal_flop().expect("flop should be dealt");
        assert!(table.is_flop());

        table.deal_turn().expect("turn should be dealt");
        assert!(table.is_turn());

        table.deal_river().expect("river should be dealt");
        assert!(table.is_river());
        assert_eq!(5, table.board.len());

        println!("{table}");

    }

}