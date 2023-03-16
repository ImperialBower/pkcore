use pkcore::play::game::Game;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::{PKError, Pile};
use pkcore::util::data::TestData;

/// cargo run --example fixed
///
/// # How it works:
/// FlopEval::try_from(game.clone()) -> CaseEvals::from_holdem_at_flop(board, &hands);
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let game = TestData::the_hand();

    println!("{}", game);

    println!();
    let flop_eval = FlopEval::try_from(game.clone()).unwrap();
    println!("{}", flop_eval);
    println!("Elapsed: {:.2?}", now.elapsed());

    // println!();
    // println!("The Nuts @ Flop:");
    // println!("{}", game.board.flop.evals());
    // println!("Elapsed: {:.2?}", now.elapsed());
    //
    // game.turn_display_odds()?;
    // println!("Elapsed: {:.2?}", now.elapsed());
    //
    // game.river_display_results();
    //
    // println!();
    // println!("{}", command(game));
    //
    // println!("Elapsed: {:.2?}", now.elapsed());

    Ok(())
}

fn _command(game: Game) -> String {
    format!(
        "cargo run --example calc -- -d  \"{}\" -b \"{}\"",
        game.hands.cards(),
        game.board.cards()
    )
}
