use pkcore::PKError;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::Table;
use pkcore::casino::table::seats::Seats;
use pkcore::util::data::TestData;

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// Season 2, Episode 11
/// `cargo run --example the_min`
fn main() -> Result<(), PKError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    // TODO: Add ante of 200
    let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));
    assert_eq!(800_000, table.table_chip_count());

    println!("{table}");
    // table.commentary_dump();

    // println!("{}", table.commentary_action_to());

    Ok(())
}
