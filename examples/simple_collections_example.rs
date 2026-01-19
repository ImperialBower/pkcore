use pkcore::prelude::*;

/// Example from the crate-level documentation.
///
/// `cargo run --example simple_collections_example`
fn main() {
    let hand: Cards = "As Ks".parse().unwrap();
    let board: Cards = "Qs Js Ts".parse().unwrap();

    // Get remaining cards
    let remaining = hand.remaining_after(&board);
    println!("Cards left in deck: {}", remaining.len());

    // Generate combinations
    for combo in hand.combinations_remaining(2) {
        println!("Possible combo: {}", Cards::from(combo));
    }
}
