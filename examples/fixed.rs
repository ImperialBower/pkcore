use pkcore::analysis::case_evals::CaseEvals;
use pkcore::play::game::Game;
use pkcore::play::stages::flop_eval::FlopEval;
use pkcore::util::data::TestData;
use pkcore::{PKError, Pile};

/// cargo run --example fixed
/// cargo run --example fixed
///
/// # How it works:
/// FlopEval::try_from(game.clone()) -> CaseEvals::from_holdem_at_flop(board, &hands);
///
/// # Deconstructing calculations
///
///
fn main() -> Result<(), PKError> {
    let now = std::time::Instant::now();
    env_logger::init();

    let game = TestData::the_hand();
    let flop_eval = FlopEval::new(game.board.flop, game.hands.clone());

    // deconstructing `let case_evals = CaseEvals::from_holdem_at_flop(board, &hands);`
    // `CaseEvals` is the struct that holds the results of the evaluation.
    let mut case_evals = CaseEvals::default();
    let cards_in_hands = game.hands.to_vec();

    // let mut minus = Cards::default();
    // let deck = Cards::deck();
    // for card in deck.iter() {
    //     if cards.get(card).is_none() {
    //         minus.insert(*card);
    //     }
    // }
    //
    // let remaining = Cards::deck_minus(&cards_in_hands)

    // First thing we need to do is figure out how many cards are remaining.
    //

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
