use bint::BintCell;
use pkcore::casino::table::Table;
use pkcore::casino::table::seats::Seats;
use pkcore::games::GamePhase;
use pkcore::util::data::TestData;

/// cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠" HSP THE HAND Negreanu/Hansen
///     https://www.youtube.com/watch?v=vjM60lqRhPg
///     https://www.youtube.com/watch?v=fEEW06iX4n8
///
/// `cargo run --example the_hand`
fn main() {
    env_logger::init();

    let mut table = Table::default();
    table.phase = GamePhase::ForcedBets.into();
    table.seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    table.button = BintCell::new(TestData::the_hand_seats().len() as u8);

    // table.deal_hole_cards().unwrap();

    println!("{table}");
}
