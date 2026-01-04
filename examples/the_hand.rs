use pkcore::PKError;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::Table;
use pkcore::casino::table::event::TableAction;
use pkcore::casino::table::seats::Seats;
use pkcore::util::data::TestData;

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// Season 2, Episode 11
/// `cargo run --example the_hand`
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // TODO: Add ante of 200
    let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
    assert_eq!(8_000_000, table.table_chip_count());

    assert!(!table.seats.all_players_have_acted());
    assert_eq!(0, table.button.value());
    assert_eq!(3, table.determine_utg());
    assert_eq!(1, table.determine_small_blind());
    assert_eq!(2, table.determine_big_blind());

    let _ = table.act_forced_bets();

    assert_eq!(8_000_000, table.table_chip_count());

    if let Some(seat) = table.get_seat(1) {
        assert_eq!(999_950, seat.player.chips.count());
        assert_eq!(50, seat.player.bet.count());
        assert_eq!(50, table.to_call(1));
    } else {
        panic!("Failed to get seat 1");
    }

    if let Some(seat) = table.get_seat(2) {
        assert_eq!(999_900, seat.player.chips.count());
        assert_eq!(100, seat.player.bet.count());
        assert_eq!(0, table.to_call(2));
    } else {
        panic!("Failed to get seat 2");
    }

    commentary_action_to(&table);

    let seat3_remaining = table.act_bet(3, 2100)?;
    assert_eq!(997_900, seat3_remaining);
    assert_eq!(table.event_log.last_player_action().unwrap(), TableAction::Bet(3, 2100));

    commentary_action_to(&table);

    let _seat4_remaining = table.act_bet(4, 5000)?;
    commentary_action_to(&table);

    let _seat5_remaining = table.act_fold(5)?;
    commentary_action_to(&table);

    let _seat6_remaining = table.act_fold(6)?;
    commentary_action_to(&table);

    let _seat7_remaining = table.act_fold(7)?;
    commentary_action_to(&table);

    let _seat0_remaining = table.act_fold(0)?;
    commentary_action_to(&table);

    let _seat1_remaining = table.act_fold(1)?;
    commentary_action_to(&table);

    let _seat2_remaining = table.act_fold(2)?;
    commentary_action_to(&table);

    //
    // println!("{table}");
    // table.commentary_dump();
    //
    // println!("{}", table.commentary_action_to());

    Ok(())
}

fn commentary_action_to(table: &Table) {
    println!();
    if let Some(action) = table.commentary_last_player_action() {
        println!("{action}");
    }
    println!("{}", table.commentary_action_to());
    println!();
}
