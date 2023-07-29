use pkcore::util::data::TestData;
use pkcore::util::wincounter::win::Win;
use pkcore::util::wincounter::PlayerFlag;
use pkcore::PKError;

/// I'm not happy with how the complexity of the code is playing out as I try to calculate
/// the player outs. Once I overcome this hump I'm feeling the need for a major refactoring.
fn main() -> Result<(), PKError> {
    let game = TestData::the_hand();

    println!("{}", game);

    // game.display_odds_at_flop()?;
    // game.display_odds_at_turn()?;

    do_it();

    Ok(())
}

fn do_it() {
    let tie = Win::FIRST | Win::THIRD;
    let mut t = tie;

    for _ in 0..8 {
        println!("{:b} {}", t, is_set(t));
        t = t >> 1;
    }
}

fn is_set(pf: PlayerFlag) -> bool {
    pf & Win::FIRST == Win::FIRST
}
