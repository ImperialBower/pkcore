use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::Table;
use pkcore::casino::table::seats::Seats;
use pkcore::util::data::TestData;

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// Season 2, Episode 11
/// `cargo run --example the_hand`
fn main() {
    env_logger::init();

    let table = Table::nlh_from_seats(
        Seats::try_from(TestData::the_hand_seats()).unwrap(),
        ForcedBets::new(50, 100),
    );

    // Doyle Brunson is the dealer.
    // Gus Hansen is under the gun.
    table.set_button(0);
    table.forced_bets();

    println!("{table}");
    println!("{}", table.event_log);
}
