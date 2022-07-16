use pkcore::util::data::TestData;
use pkcore::PKError;

fn main() -> Result<(), PKError> {
    let game = TestData::the_hand();

    println!("{}", game);

    game.display_odds_at_flop()?;

    Ok(())
}
