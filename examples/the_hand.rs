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
    //
    // let table = Table::nlh_from_seats(
    //     Seats::try_from(TestData::the_hand_seats()).unwrap(),
    //     ForcedBets::new(50, 100),
    // );
    //
    // // Doyle Brunson is the dealer.
    // // Gus Hansen is under the gun.
    // table.button_set(0);
    // table.act_shuffle_deck();
    // let _ = table.act_forced_bets();
    //
    // println!("{table}");
    // println!("{}", table.event_log);
    // TODO: Add ante of 200
    let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
    assert_eq!(800_000, table.table_chip_count());
    assert_eq!(0, table.button.value());
    assert_eq!(3, table.determine_utg());
    assert_eq!(1, table.determine_small_blind());
    assert_eq!(2, table.determine_big_blind());

    // table.act_button_move();
    // assert_eq!(1, table.button.value());
    // assert_eq!(4, table.determine_utg());
    // assert_eq!(2, table.determine_small_blind());
    // assert_eq!(3, table.determine_big_blind());

    let _ = table.act_forced_bets();
    assert_eq!(800_000, table.table_chip_count());

    if let Some(seat) = table.seat(1) {
        assert_eq!(99_950, seat.player.chips.count());
        assert_eq!(50, seat.player.bet.count());
        assert_eq!(50, table.to_call(1));
    } else {
        panic!("Failed to get seat 1");
    }

    if let Some(seat) = table.seat(2) {
        assert_eq!(99_900, seat.player.chips.count());
        assert_eq!(100, seat.player.bet.count());
        assert_eq!(0, table.to_call(2));
    } else {
        panic!("Failed to get seat 2");
    }

    if let Some(seat) = table.seat(6) {
        assert_eq!(100_000, seat.player.chips.count());
        assert_eq!(0, seat.player.bet.count());
        assert_eq!(100, table.to_call(6));
    } else {
        panic!("Failed to get seat 6");
    }

    println!("{}", table.commentary_action_to());

    let seat3_remaining = table.act_bet(3, 2100)?;
    assert_eq!(97_900, seat3_remaining);
    assert_eq!(table.event_log.last().unwrap(), TableAction::Bet(3, 2100));

    let _seat4_remaining = table.act_bet(4, 5000)?;
    let _seat5_remaining = table.act_fold(5)?;
    let _seat6_remaining = table.act_fold(6)?;
    let _seat7_remaining = table.act_fold(7)?;

    println!("{table}");
    table.commentary_dump();

    println!("{}", table.commentary_action_to());

    Ok(())
}
