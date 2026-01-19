use pkcore::prelude::*;

/// Example from the crate-level documentation.
///
/// `cargo run --example simple_eval_example`
fn main() {
    // 1st player has A♠ K♥ while 2nd player has 8♦ K♣
    let hands = HoleCards::from_str("A♠ KH 8♦ K♣").unwrap();
    let board = Board::from_str("A♣ 8♥ 7♥ 9♠").unwrap();
    let game = Game::new(hands, board);

    let case_evals = game.turn_case_evals();
    let outs = Outs::from(&case_evals);

    let player1_outs = outs.get(1).unwrap();
    let player2_outs = outs.get(2).unwrap();

    // Show the outs for each player
    println!("Player #1 has {} outs: {}", player1_outs.len(), player1_outs);
    println!("Player #2 has {} outs: {}", player2_outs.len(), player2_outs);

    // Show each players odds of winning against every possible card dealt as well
    // as their best hand at the turn.
    game.turn_display_odds().expect("TurnDisplayOdds");
}
