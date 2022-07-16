use pkcore::util::data::TestData;
use pkcore::PKError;

/// I'm not happy with how the complexity of the code is playing out as I try to calculate
/// the player outs. Once I overcome this hump I'm feeling the need for a major refactoring.
fn main() -> Result<(), PKError> {
    let game = TestData::the_hand();

    println!("{}", game);

    game.display_odds_at_flop()?;
    game.display_odds_at_turn()?;

    Ok(())
}
